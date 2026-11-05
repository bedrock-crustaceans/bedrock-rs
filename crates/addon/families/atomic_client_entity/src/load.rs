//! Loading atomic client entities: the beta-only chain, `Content` / `Definition`, the walk of
//! `atomic_client_entities/` and the combine (the last document of an identifier wins whole).

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{AtomicClientEntityId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, PrepareError, Prepared, Value, WriteStyle, last_wins, write_json,
};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BETA_VERSION, BetaPermission, ErrorMask, Family, InputKind, Loaded, MissingVersion,
    NoComponents, NoStable, StructIndex, load,
};

use super::AtomicClientEntity;

/// The directory atomic client entity files live in.
const DIRECTORY: &str = "atomic_client_entities";
/// The payload key.
const PAYLOAD: &str = "minecraft:atomic_client_entity";

/// The atomic client entity family of the version-chain machinery (hidden: only the loader
/// names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct AtomicFamily;

bedrock_addon_core::version_chain! {
    family = AtomicFamily;
    /// An atomic client entity document as parsed: the beta struct is the only one.
    #[doc(hidden)]
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {}
    beta =
        /// `"format_version": "beta"`.
        Beta(AtomicClientEntity);
}

impl Family for AtomicFamily {
    const NAME: &'static str = "atomic_client_entity";
    const PAYLOAD_KEY: &'static str = PAYLOAD;
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    // Unused: there is no stable struct, so every version but `"beta"` has no parser.
    const FLOOR: FormatVersion = FormatVersion::new(0, 0, 0);
    type Latest = NoStable;
    type Beta = AtomicClientEntity;
    // The game builds the load context with the beta permission on, whatever the pack and the
    // experiments.
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    // The one component is a member of `Components`; the generic component checks run on a
    // stable struct only, which this family does not have.
    type Component = NoComponents;
    type Any = Any;
    // A document without a version fails with "no valid version passed by the caller or found in
    // the input".
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // An empty file is skipped silently.
    const EMPTY_DOCUMENT: Severity = Severity::Info;
    // The game reads the file through the document-input reader of the cereal-based families:
    // the version is read from the document, and a member written `null` fails its type (the
    // reader applies that rule itself).
    const INPUT: InputKind = InputKind::Document;

    // The one slice is registered with the strict error mask, so every cereal finding is fatal.
    // The reader reports unknown members and components as errors itself; the mask says the
    // same.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }
}

impl Sealed for AtomicClientEntity {}

impl Content for AtomicClientEntity {
    const NAME: &'static str = "atomic client entity";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Parses one atomic client entity file. Only `"format_version": "beta"` loads.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Resource, path, bytes, cx).1
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let mut root = facet_value::VObject::new();
        root.insert("format_version", "beta");
        root.insert(
            PAYLOAD,
            bedrock_addon_core::versioning::write_payload(self)?,
        );
        write_json(&Value::from(root), out, style)
    }
}

impl JsonContent for AtomicClientEntity {}

impl Definition for AtomicClientEntity {
    type Kind = kind::AtomicClientEntity;
    type File = AtomicClientEntity;
    type Beta = NoBeta<kind::AtomicClientEntity>;
}

impl Placed for AtomicClientEntity {
    /// `atomic_client_entities/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

/// Gives every finding the document's pack and file, and the envelope's findings the game's
/// texts: a missing payload its text, and a malformed version the "no valid version" line that
/// follows it.
fn place(found: Diagnostics, origin: &Origin) -> Diagnostics {
    let mut placed = Diagnostics::default();
    for mut finding in found {
        let malformed = matches!(finding.code, DiagnosticCode::MalformedFormatVersion { .. });
        if matches!(finding.code, DiagnosticCode::MissingPayload { text: None }) {
            // A missing payload gets the game's text.
            finding.code = bedrock_addon_core::diagnostic::cereal::payload_missing();
        }
        let location = finding.location.clone();
        placed.push(finding);
        if malformed {
            // The game writes "Malformed format version", then the "no valid version" text.
            let mut missing = Diagnostic::error(DiagnosticCode::MissingFormatVersion {
                text: Some(VanillaText::NoValidVersion),
            });
            missing.location = location;
            placed.push(missing);
        }
    }
    placed.located(origin)
}

/// The envelope's member-count rule for a root without `format_version`: a root that holds
/// only the payload fails "exactly two members" before the missing version is looked at.
fn member_count(text: &str) -> Option<Diagnostic> {
    let cleaned = last_wins(text);
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(&cleaned).ok()?;
    let root = root.as_object()?;
    (root.get("format_version").is_none() && root.len() != 2).then(|| {
        Diagnostic::error(DiagnosticCode::MalformedInput {
            message: format!(
                "{} root members, not \"format_version\" and a payload member",
                root.len()
            ),
            text: Some(VanillaText::DocumentMemberCount),
        })
    })
}

/// Loads one atomic client entity file: the document or its rejection.
fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (
    Option<AtomicClientEntityId>,
    Result<Document<AtomicClientEntity>, Rejected>,
) {
    let origin = Origin::file(pack, side, path.clone());
    let reject = |found: Diagnostic| {
        let diagnostics: Diagnostics = [found].into_iter().collect();
        Rejected::new(origin.clone(), place(diagnostics, &origin))
    };
    let malformed = |message: &str| {
        reject(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: message.to_owned(),
            text: None,
        }))
    };
    let prepared = match Prepared::new(bytes) {
        Ok(prepared) => prepared,
        Err(PrepareError::NotUtf8(_)) => {
            return (None, Err(malformed("the file is not UTF-8")));
        }
        Err(PrepareError::TooDeep(_) | PrepareError::Empty) => {
            // `new` never refuses empty input.
            return (None, Err(malformed("the JSON nests too deeply")));
        }
    };
    let text = prepared.as_str();
    if let Some(found) = member_count(text) {
        return (None, Err(reject(found)));
    }
    let outcome = load::<AtomicFamily>(JsonSlice::new(text), cx);
    let diagnostics = place(outcome.diagnostics, &origin);
    match outcome.value {
        Some(Loaded::Beta(entity)) => {
            let id = entity.description.identifier.clone();
            let mut version = outcome.version;
            version.parsed_as = BETA_VERSION;
            (
                Some(id),
                Ok(Document::new(entity, origin, version, diagnostics)),
            )
        }
        _ => (None, Err(Rejected::new(origin, diagnostics))),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `atomic_client_entities/**/*.json`, packs from the lowest priority to the highest: the
/// extension is exactly `json`; any other file of the directory is skipped silently.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]);

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<AtomicClientEntity> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        (Some(id), Ok(document)) => Contribution::Stable(id, document),
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
        // A loaded document always has its identifier.
        (None, Ok(document)) => {
            Contribution::Stable(document.description.identifier.clone(), document)
        }
    }
}

/// One definition per identifier: every loaded document assigns its identifier, so the last
/// one in walk order replaces the earlier one whole (the highest-priority pack's, and inside a
/// pack the last file's). No member is merged and `min_engine_version` plays no part. Each
/// replaced document gets an `AtomicClientEntityShadowed` info (the game is silent here, so
/// the finding is the crate's own). The registry assigns by identifier, so the later document
/// replaces the earlier one.
fn combine(
    parsed: Vec<Contribution<AtomicClientEntity>>,
    stack: &StackCx<'_>,
) -> Registry<AtomicClientEntity> {
    let mut builder = RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice {
                code: DiagnosticCode::AtomicClientEntityShadowed,
                severity: Severity::Info,
            }),
        },
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in parsed {
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `rp.atomic_client_entities()`: `atomic_client_entities/**/*.json` of every
/// resource pack, the last document of an identifier winning.
pub const fn load_registry()
-> CollectionLoader<Registry<AtomicClientEntity>, Contribution<AtomicClientEntity>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
