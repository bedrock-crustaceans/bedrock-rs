//! Server sound definitions: a behaviour pack's `sounds/sound_definitions.json` with the
//! `minecraft:server_sound_definitions` payload, the metadata the server keeps about a sound
//! it plays. One stable struct at exactly 1.26.60; no other version, `"beta"` included, has a
//! parser. The file is read strictly (an
//! unknown member, a wrong type, a `null` member or a failed constraint rejects the whole
//! file).

#![cfg_attr(docsrs, feature(doc_cfg))]

mod read;
mod types;
mod validate;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase, VanillaText};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, JsonDocument, JsonPlaced, NoBeta, Origin,
    Registry, RegistryBuilder, Rejected,
};
use bedrock_addon_core::id::{SoundEventName, kind};
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{
    self, Accept, CollectionLoader, FileCopy, LoadContext, StackCx, Walk,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, BeyondWidth, DocumentRead, Envelope, EnvelopeCheck, Family, InputKind, Loaded,
    MissingVersion, NoComponents, Reading, Scalars, Single, StructIndex, ValidateCx,
    parse_document, parse_document_as_written,
};

pub use types::{
    DurationInfo, DurationMode, MusicInfo, SoundDuration, SoundMetadata, SoundMetadataFile,
    TagValueList,
};

/// The version of the one struct, and of the one slice: the game's document format is
/// 1.26.60.
pub(crate) const V1_26_60: FormatVersion = bedrock_addon_core::version::known::V1_26_60;

/// The engine version whose struct first binds `broadcast_range`; an older engine does not
/// know the member.
pub(crate) const BROADCAST_RANGE_SINCE: FormatVersion =
    bedrock_addon_core::version::known::V1_26_50;

/// The server sound definitions family of the version-chain machinery.
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct MetadataFamily;

/// The file parsed by the one struct, not upgraded.
pub(crate) type MetadataAny = Single<MetadataFamily>;

impl Family for MetadataFamily {
    const NAME: &'static str = "server_sound_definitions";
    const PAYLOAD_KEY: &'static str = "minecraft:server_sound_definitions";
    // The one slice covers `[1.26.60, end]`, where the end is the larger of the slice's version
    // and the game's current version, which is 1.26.60: the engine version closes it.
    const FLOOR: FormatVersion = V1_26_60;
    const ENVELOPE: Envelope = Envelope {
        // The loader reads the document as JSON input: the version as the game reads it from the
        // text, and `null` members rejected.
        input: InputKind::Document,
        // The envelope is checked only at or above `minModernVersion`, the slice's version.
        check: EnvelopeCheck::From(V1_26_60),
        // The game's loader is given no caller version, so a file without `format_version` is
        // rejected.
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = SoundMetadataFile;
    // No Beta slice: `"beta"` has no parser.
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Single<Self>;

    /// The document-input profile, its scalars read strictly: a member takes its own JSON type
    /// alone, and a number beyond its float is a type error.
    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        MetadataFamily::ENVELOPE
            .input
            .profile()
            .with_scalars(Scalars::Strict)
            .with_beyond_width(BeyondWidth::Error)
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        let Loaded::Stable(file) = v else { return };
        validate::validate(file, cx);
    }
}

/// The one path the server reads the definitions of a behaviour pack from.
/// The game names this one file; it never lists the directory.
pub(crate) const METADATA_PATH: &str = "sounds/sound_definitions.json";

/// Vanilla's text for a sound event name that is registered already: an error in the sound
/// log area, printed once.
const DUPLICATE: &str =
    "Duplicate server sound definition for sound event '{}'; the later definition will be ignored.";

impl bedrock_addon_core::__private::Sealed for SoundMetadata {}

/// `read` of the text the family reads: of a member written twice, the last copy is the one
/// that stands (`json::last_wins` of the file's text). Bytes that are no text are passed on as
/// they are, for the generic read to reject.
fn with_last_wins<R>(bytes: &[u8], read: impl FnOnce(&[u8]) -> R) -> R {
    // A leading byte-order mark is blanked, not cut, so every span is an offset into the file
    // as written.
    match Prepared::new(bytes) {
        Ok(prepared) => read(bedrock_addon_core::json::last_wins(prepared.as_str()).as_bytes()),
        Err(_) => read(bytes),
    }
}

impl JsonDocument for SoundMetadataFile {
    type Family = MetadataFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Reads a behaviour pack's `sounds/sound_definitions.json`. Only `"format_version":
    /// "1.26.60"` is read, by any pack (no toggle); the file is rejected as a whole for any
    /// error in it.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<MetadataFamily> {
        with_last_wins(bytes, |text| {
            parse_document::<MetadataFamily>(origin, text, cx)
        })
    }

    /// The struct the version selects, read from the same text as [`read`](Self::read).
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<MetadataAny>, Rejected> {
        with_last_wins(bytes, |text| {
            parse_document_as_written::<MetadataFamily>(PackSide::Behavior, path, text, cx)
        })
    }
}

impl JsonPlaced for SoundMetadataFile {
    /// `sounds/sound_definitions.json` of the behaviour pack: the one path the server reads.
    fn place(&self) -> PackPath {
        PackPath::from_static(METADATA_PATH)
    }
}

/// An entry of the file: a hand `Definition`, as the file holds several.
impl Definition for SoundMetadata {
    type Kind = kind::SoundEvent;
    type File = SoundMetadataFile;
    type Beta = NoBeta<kind::SoundEvent>;
}

/// The walk of `bp.server_sound_definitions()`: the fixed file of each pack, lowest pack first as
/// vanilla visits the stack.
const WALK: Walk = Walk::json_children_in(&["sounds"]).accept(Accept::Exact(&[METADATA_PATH]));

/// The duplicate finding of the definition at `index` of `file`.
fn duplicate(file: &Origin, name: &str, index: usize) -> Diagnostic {
    let at = JsonPath::new()
        .child(MetadataFamily::PAYLOAD_KEY)
        .index(index);
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
    parsed: Vec<FileCopy<SoundMetadataFile>>,
    stack: &StackCx<'_>,
) -> Registry<SoundMetadata> {
    let mut builder = RegistryBuilder::new(
        CombineRule::bottom_wins(),
        CombineRule::bottom_wins(),
        stack.options,
    );
    let mut stored: IndexMap<String, PackRef> = IndexMap::new();
    for (_, file) in parsed {
        let file = match file {
            Ok(file) => file,
            Err(rejected) => {
                builder.push(Contribution::Rejected(None, rejected));
                continue;
            }
        };
        let (origin, version) = (file.origin().clone(), file.version().clone());
        let mut file_findings = Some(file.diagnostics().clone());
        let mut rows = file.unknown().clone();
        for (index, definition) in file.into_value().0.into_iter().enumerate() {
            // The members the file kept under the definition are its document's, at its root.
            let at = JsonPath::new()
                .child(MetadataFamily::PAYLOAD_KEY)
                .index(index);
            let unknown = rows.take_below(&at);
            let name = definition.sound_event_name.clone();
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
                    let document = Document::new(definition, entry, version.clone(), diagnostics)
                        .with_unknown_members(unknown);
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
                    let document = Document::new(definition, entry, version.clone(), diagnostics)
                        .with_unknown_members(unknown);
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

/// The loader of `bp.server_sound_definitions()`: each pack's file, parsed whole, and the
/// family's combine into one registry of its definitions (the file holds several).
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<SoundMetadata>, FileCopy<SoundMetadataFile>> =
    load::merged::<SoundMetadataFile, _>(WALK, combine);

#[cfg(test)]
mod tests {
    use super::validate::{NEEDS_NAMESPACE, namespace_text, no_whitespace};
    use super::*;
    use bedrock_addon_core::json::NullRule;
    use bedrock_addon_core::load::DocumentSource;
    use bedrock_addon_core::version::FormatVersionInput;
    use bedrock_addon_core::versioning::{
        ErrorMask, InputKind, Selection, StructIndex, VersionChain, select,
    };

    /// The `to_json` the family once wrote by hand, the chain's text as is, equals `Content`'s
    /// default, which lays that text out again: reals included.
    #[test]
    fn the_default_to_json_is_the_chain_text_as_is() {
        use bedrock_addon_core::document::Content;
        use bedrock_addon_core::load::LoadContext;
        use bedrock_addon_core::pack_identity::PackPath;
        let text = r#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": [
            {"sound_event_name": "demo:x", "duration_info": {"mode": "real_time", "seconds": 1.2345},
             "broadcast_range": 64.0},
            {"sound_event_name": "demo:y", "duration_info": {"mode": "game_time", "seconds": 0.1},
             "broadcast_range": 16.5}
        ]}"#;
        let document = <SoundMetadataFile as Content>::parse(
            &PackPath::from_static("sounds/sound_definitions.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("loads");
        let file = document.value();
        assert_eq!(
            file.to_json().unwrap(),
            Single::<MetadataFamily>(file.clone()).to_json().unwrap()
        );
    }

    #[test]
    fn the_chain_is_the_stable_struct_alone() {
        // One slice at 1.26.60 with the strict mask, then the engine version closes it; there
        // is no beta slice.
        assert_eq!(
            <MetadataAny as VersionChain<MetadataFamily>>::VERSIONS,
            [FormatVersion::new(1, 26, 60)]
        );
        const { assert!(!<MetadataAny as VersionChain<MetadataFamily>>::HAS_BETA) };
        assert_eq!(MetadataFamily::ENVELOPE.input, InputKind::Document);
        assert_eq!(
            MetadataFamily::reading(StructIndex::Stable(0), DocumentSource::Json).null_rule(),
            Some(NullRule::Rejected)
        );
        assert_eq!(
            MetadataFamily::reading(StructIndex::Stable(0), DocumentSource::Json).error_mask(),
            ErrorMask::Strict
        );
        assert_eq!(
            MetadataFamily::PAYLOAD_KEY,
            "minecraft:server_sound_definitions"
        );
    }

    #[test]
    fn only_the_current_format_selects_the_struct() {
        let selects =
            |text: &str| select::<MetadataFamily>(&FormatVersionInput::parse(text).unwrap());
        assert_eq!(selects("1.26.60"), Selection::Struct(0));
        for other in [
            "1.26.40",
            "1.26.50",
            "1.26.60-beta.1",
            "1.26.61",
            "9999.9999.9999",
        ] {
            assert!(!matches!(selects(other), Selection::Struct(_)), "{other}");
        }
    }

    #[test]
    fn the_namespace_rule_follows_the_two_regexes() {
        let text = |name: &str| namespace_text(name).map(|text| text.to_string());
        assert_eq!(text("demo:click"), None);
        assert_eq!(text("a:b"), None);
        assert_eq!(text(":a:b"), None);
        for unprefixed in ["click", ":click", "demo:", ":", "", "minecraft:"] {
            assert_eq!(
                text(unprefixed).as_deref(),
                Some(NEEDS_NAMESPACE),
                "{unprefixed}"
            );
        }
        for disallowed in ["minecraft:a", "MineCraft:a"] {
            assert_eq!(
                text(disallowed).as_deref(),
                Some("string has invalid namespace (minecraft)"),
                "{disallowed}"
            );
        }
    }

    #[test]
    fn the_whitespace_pattern_is_non_empty_and_space_free() {
        assert!(no_whitespace("demo:x"));
        for bad in [
            "",
            " ",
            "a b",
            "a\tb",
            "a\n",
            "\u{a0}a",
            "a\u{2003}b",
            "a\u{feff}",
        ] {
            assert!(!no_whitespace(bad), "{bad:?}");
        }
    }

    /// A definition's document holds the unknown members the file kept under it, by paths
    /// from the definition: what it writes as its own document.
    #[test]
    fn a_definition_holds_the_unknown_members_under_it() {
        use bedrock_addon_core::document::Content;
        use bedrock_addon_core::json::Value;
        use bedrock_addon_core::load::LoadOptions;
        let text = r#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": [
            {"sound_event_name": "demo:x", "duration_info": {"mode": "real_time", "seconds": 1}},
            {"sound_event_name": "demo:y", "duration_info": {"mode": "game_time", "seconds": 2}}
        ]}"#;
        let path = PackPath::from_static("sounds/sound_definitions.json");
        let mut file =
            <SoundMetadataFile as Content>::parse(&path, text.as_bytes(), &LoadContext::detached())
                .expect("loads");
        // A file read strictly keeps none; the rows are an edit's.
        let second = JsonPath::new().child(MetadataFamily::PAYLOAD_KEY).index(1);
        file.unknown_mut()
            .put(&second, "zz_definition", Value::from(1i64));
        file.unknown_mut().put(
            &second.child("duration_info"),
            "zz_duration",
            Value::from(2i64),
        );
        let options = LoadOptions::default();
        let registry = combine(vec![(path, Ok(file))], &StackCx::new(&options, &[]));
        let x = registry
            .get(&SoundEventName::parse("demo:x"))
            .expect("demo:x");
        assert!(x.unknown().is_empty(), "{:?}", x.unknown());
        let y = registry
            .get(&SoundEventName::parse("demo:y"))
            .expect("demo:y");
        let rows: Vec<(String, &str)> = y
            .unknown()
            .iter()
            .map(|(at, key, _)| (at.to_string(), key))
            .collect();
        assert_eq!(
            rows,
            [
                (String::new(), "zz_definition"),
                ("duration_info".to_owned(), "zz_duration")
            ]
        );
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn server_sound_definitions_reads_as_pinned() {
        assert_reading_table(
            "server_sound_definitions::MetadataFamily",
            &reading_table::<super::MetadataFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 26, patch: 60 })
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.26.60..=1.26.60 (0..=0): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }
}
