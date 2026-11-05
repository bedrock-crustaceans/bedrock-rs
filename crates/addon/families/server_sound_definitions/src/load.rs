//! Loading the server sound definitions: the one fixed file of each behaviour pack, and the
//! registry of the stack, in which the lowest pack's definition of a name is the one kept.

use indexmap::IndexMap;

use super::{MetadataAny, MetadataFamily, SoundMetadata, SoundMetadataFile};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Origin, Placed,
    Registry, RegistryBuilder, Rejected,
};
use bedrock_addon_core::id::{SoundEventName, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Family, Loaded, load};

/// The one path the server reads the definitions of a behaviour pack from.
/// The game names this one file; it never lists the directory.
pub(crate) const METADATA_PATH: &str = "sounds/sound_definitions.json";

/// Vanilla's text for a sound event name that is registered already: an error in the sound
/// log area, printed once.
const DUPLICATE: &str =
    "Duplicate server sound definition for sound event '{}'; the later definition will be ignored.";

impl bedrock_addon_core::__private::Sealed for SoundMetadataFile {}
impl bedrock_addon_core::__private::Sealed for SoundMetadata {}

/// Gives every finding the file's pack and path.
fn locate(diagnostics: Diagnostics, origin: &Origin) -> Diagnostics {
    let mut out = Diagnostics::default();
    for mut finding in diagnostics {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        out.push(finding);
    }
    out
}

/// Gives a version below the one slice this loader's text: the versioned loader has no parser below
/// its first slice, so a version below 1.26.60 is "no parser
/// available" like one above it, not the "use json format version … or higher" of a floor.
fn vanilla_texts(diagnostics: Diagnostics, written: Option<FormatVersion>) -> Diagnostics {
    let mut out = Diagnostics::default();
    for mut finding in diagnostics {
        if let DiagnosticCode::UnsupportedVersion { text } = &mut finding.code
            && matches!(text, Some(VanillaText::UseFormatVersion { .. }))
            && let Some(version) = written
        {
            *text = Some(VanillaText::NoParserForVersion {
                version: version.to_string(),
            });
        }
        out.push(finding);
    }
    out
}

/// Loads the file of `pack` at `path`.
fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<SoundMetadataFile>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    // A leading byte-order mark is blanked, not cut, so every span is an offset into the file
    // as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    let text = check_depth(bytes, MAX_JSON_DEPTH)
        .ok()
        .and_then(|()| std::str::from_utf8(bytes).ok());
    let Some(text) = text else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: "not UTF-8 JSON within the nesting limit".to_owned(),
            text: None,
        }));
        return Err(Rejected::new(origin.clone(), locate(diagnostics, &origin)));
    };
    let text = bedrock_addon_core::json::last_wins(text);
    let document = JsonSlice::new(&text);
    let outcome = load::<MetadataFamily>(document, cx);
    let diagnostics = locate(
        vanilla_texts(outcome.diagnostics, outcome.version.written),
        &origin,
    );
    match outcome.value {
        Some(Loaded::Stable(file)) => Ok(Document::new(file, origin, outcome.version, diagnostics)),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

impl Content for SoundMetadataFile {
    const NAME: &'static str = MetadataFamily::NAME;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Reads a behaviour pack's `sounds/sound_definitions.json`. Only `"format_version":
    /// "1.26.60"` is read, by any pack (no toggle); the file is rejected as a whole for any
    /// error in it.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let text = MetadataAny::V1_26_60(self.clone()).to_json()?;
        // The document is written once and laid out in `style`.
        if let Ok(document) =
            bedrock_addon_core::json::from_str::<bedrock_addon_core::json::Value>(&text)
        {
            bedrock_addon_core::json::write_json(&document, out, style)
        } else {
            out.extend_from_slice(text.as_bytes());
            Ok(())
        }
    }
}

impl JsonContent for SoundMetadataFile {}

impl Placed for SoundMetadataFile {
    /// `sounds/sound_definitions.json` of the behaviour pack: the one path the server reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(METADATA_PATH)
    }
}

impl Definition for SoundMetadata {
    type Kind = kind::SoundEvent;
    type File = SoundMetadataFile;
    type Beta = NoBeta<kind::SoundEvent>;
}

/// The walk's file.
fn accepts(path: &PackPath) -> bool {
    path.as_str() == METADATA_PATH
}

/// The walk of `bp.server_sound_definitions()`: the fixed file of each pack, lowest pack first as
/// vanilla visits the stack.
const WALK: Walk = Walk {
    directories: &["sounds"],
    depth: ListDepth::Children,
    accept: Accept::Fn(accepts),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// Parses one walked file.
fn parse_walk_file(
    file: &WalkFile,
    cx: &LoadContext,
) -> Result<Document<SoundMetadataFile>, Rejected> {
    parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx)
}

/// The duplicate finding of the definition at `index` of `file`.
fn duplicate(file: &Origin, name: &str, index: usize) -> Diagnostic {
    let mut at = JsonPath::new();
    at.push(MetadataFamily::PAYLOAD_KEY);
    at.push_index(index);
    Diagnostic::error(DiagnosticCode::DuplicateIdentifier {
        text: Some(VanillaText::Formatted {
            template: DUPLICATE,
            args: vec![name.to_owned()],
        }),
    })
    .at(at)
    .phase(Phase::Validate)
    .in_file(file)
}

/// Combines the files, lowest pack first: the first definition of a name is kept, compared
/// byte for byte; every later one is dropped with vanilla's duplicate error (raised for each,
/// where vanilla prints an identical text once).
fn combine(
    parsed: Vec<Result<Document<SoundMetadataFile>, Rejected>>,
    stack: &StackCx<'_>,
) -> Registry<SoundMetadata> {
    let mut builder = RegistryBuilder::new(
        CombineRule::bottom_wins(),
        CombineRule::bottom_wins(),
        stack.options,
    );
    let mut stored: IndexMap<String, PackRef> = IndexMap::new();
    for file in parsed {
        let file = match file {
            Ok(file) => file,
            Err(rejected) => {
                builder.push(Contribution::Rejected(None, rejected));
                continue;
            }
        };
        let (origin, version) = (file.origin().clone(), file.version().clone());
        let mut file_findings = Some(file.diagnostics().clone());
        for (index, definition) in file.into_value().0.into_iter().enumerate() {
            let name = definition.sound_event_name.0.clone();
            let id = SoundEventName::parse(name.as_str());
            let entry = Origin::new(
                origin.pack.clone(),
                origin.side,
                origin.path.clone(),
                Some(name.clone().into_boxed_str()),
                Vec::new(),
            );
            // The file's own findings travel with its first definition.
            let mut diagnostics = file_findings.take().unwrap_or_default();
            match stored.get(&name) {
                None => {
                    stored.insert(name, origin.pack.clone());
                    let document = Document::new(definition, entry, version.clone(), diagnostics);
                    builder.push(Contribution::Stable(id, document));
                }
                Some(first) if *first == origin.pack => {
                    diagnostics.push(duplicate(&origin, &name, index));
                    builder.push(Contribution::Rejected(
                        Some(id),
                        Rejected::new(entry, diagnostics),
                    ));
                }
                Some(_) => {
                    let document = Document::new(definition, entry, version.clone(), diagnostics);
                    builder.push(Contribution::Stable(id, document));
                    builder.push_diagnostic(duplicate(&origin, &name, index));
                }
            }
        }
        if let Some(findings) = file_findings {
            for finding in findings {
                builder.push_diagnostic(finding);
            }
        }
    }
    builder.finish()
}

/// The loader of `bp.server_sound_definitions()`.
pub const fn load_registry()
-> CollectionLoader<Registry<SoundMetadata>, Result<Document<SoundMetadataFile>, Rejected>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
