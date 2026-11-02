//! The steps of loading one spawn file that spawn rules and spawn groups share: the text check,
//! the routing around a family's load, the rejection lines, and the combine rule of the
//! registries.

use super::texts;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, Origin, Registry, RegistryBuilder, Rejected,
};
use bedrock_addon_core::json::{JsonSlice, MAX_JSON_DEPTH, Value, check_depth, strip_bom};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::load::StackCx;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Family, Loaded, finish, parse_as_written};

/// A spawn file loaded: the identifier its payload names (when it was read), the document or
/// its rejection, and whether vanilla registers it.
#[derive(Clone, Debug)]
pub struct FileLoad<T> {
    /// The identifier the payload names, when it was read.
    pub identifier: Option<String>,
    /// The document, or why the file was rejected.
    pub result: Result<Document<T>, Rejected>,
    /// Whether vanilla registers the file.
    pub registered: bool,
}

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
pub fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// A rejection finding at the document root.
pub fn error(code: DiagnosticCode) -> Diagnostic {
    Diagnostic::new(code, Severity::Error, Phase::Parse, Location::default())
}

/// The first member named `format_version` in the document, depth first in document order:
/// vanilla's raw scan for the version.
pub(crate) fn first_format_version(value: &Value) -> Option<&Value> {
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            if key.as_str() == "format_version" {
                return Some(member);
            }
            if let Some(found) = first_format_version(member) {
                return Some(found);
            }
        }
    }
    value.as_array()?.iter().find_map(first_format_version)
}

/// What the raw path makes of a root that is no JSON object: with no `format_version` anywhere
/// in the text, the loader's "no valid version" error; otherwise the parser's "Spawn rules json was
/// not formatted as an object".
pub(crate) fn not_an_object(root: &Value) -> Diagnostic {
    if first_format_version(root).is_none() {
        error(DiagnosticCode::MissingFormatVersion {
            text: Some(VanillaText::NoValidVersion),
        })
    } else {
        error(DiagnosticCode::MalformedInput {
            message: "the root is not an object".to_owned(),
            text: Some(VanillaText::Fixed(texts::NOT_AN_OBJECT)),
        })
    }
}

/// The text of a rejected document's closing line.
pub fn closing(text: &'static str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::DocumentRejected {
            text: Some(VanillaText::Fixed(text)),
        },
        Severity::Error,
        Phase::Validate,
        Location::default(),
    )
}

/// Gives every finding the document's pack and file. The versioned loader's text replaces
/// the floor text of a version below 1.8.0 (the game's legacy slices have no floor message:
/// "no parser available"); the checked envelope's missing payload takes the document-input
/// text of the game's loader.
pub fn locate(diagnostics: &mut Diagnostics, origin: &Origin, written: Option<FormatVersion>) {
    // In place: the list keeps where the members of the document are in the file.
    for finding in diagnostics.iter_mut() {
        if matches!(finding.code, DiagnosticCode::MissingPayload { text: None }) {
            finding.code = bedrock_addon_core::diagnostic::cereal::payload_missing();
        }
        if let DiagnosticCode::UnsupportedVersion {
            text: text @ Some(VanillaText::UseFormatVersion { .. }),
        } = &mut finding.code
        {
            *text = Some(VanillaText::NoParserForVersion {
                version: written.map(|v| v.to_string()).unwrap_or_default(),
            });
        }
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
    }
}

/// One family's load of `text`, mapped to the root type.
pub fn load_family<F: Family, T>(
    text: &str,
    cx: &LoadContext,
    origin: &Origin,
    map: fn(F::Latest) -> T,
) -> (Option<String>, Result<Document<T>, Rejected>) {
    let parsed = parse_as_written::<F>(JsonSlice::new(text), cx);
    let identifier = parsed.identity.identifier.clone();
    let written = parsed.version.written;
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    locate(&mut diagnostics, origin, written);
    let result = match outcome.value {
        Some(Loaded::Stable(value)) => Ok(Document::new(
            map(value),
            origin.clone(),
            outcome.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin.clone(), diagnostics)),
    };
    (identifier, result)
}

/// Loads one spawn file of either kind: the shared steps before and after the family's load.
pub fn load_file<T>(
    origin: &Origin,
    bytes: &[u8],
    closing_text: &'static str,
    load: impl FnOnce(&str, &Origin) -> (Option<String>, Result<Document<T>, Rejected>),
) -> (Option<String>, Result<Document<T>, Rejected>) {
    // The BOM is blanked, not cut, so every offset is an offset into the file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let reject = |finding: Diagnostic, closes: bool| {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(finding);
        if closes {
            diagnostics.push(closing(closing_text));
        }
        locate(&mut diagnostics, origin, None);
        Err(Rejected::new(origin.clone(), diagnostics))
    };
    let Some(text) = text(bytes) else {
        let malformed = error(DiagnosticCode::MalformedInput {
            message: "not UTF-8 JSON within the nesting limit".to_owned(),
            text: None,
        });
        return (None, reject(malformed, false));
    };
    // A JSON syntax error: vanilla's parser returns an empty document without a message, and no
    // closing line; this crate rejects it with the reader's finding, from the family's load.
    let syntax_error = match bedrock_addon_core::json::from_str_jsonc::<Value>(text) {
        Ok(root) if root.as_object().is_none() && !root.is_null() => {
            return (None, reject(not_an_object(&root), true));
        }
        Ok(root) => root.is_null(),
        Err(_) => true,
    };
    let (identifier, mut result) = load(text, origin);
    if !syntax_error && let Err(rejected) = &mut result {
        let mut closing = closing(closing_text);
        closing.location.pack = Some(origin.pack.clone());
        closing.location.path = Some(origin.path.clone());
        rejected.diagnostics.push(closing);
    }
    (identifier, result)
}

/// The origin of a file of `pack`.
pub fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::new(pack, side, path.clone(), None, Vec::new())
}

/// What one file adds to a registry: a contribution, or the findings of a document vanilla does
/// not register.
#[derive(Debug)]
pub enum Parsed<T: Definition> {
    /// A registered document or a rejection.
    Contribution(Contribution<T>),
    /// A document that loaded but is not registered.
    Unregistered(Diagnostics),
}

/// The contribution of a loaded file, filed under the identifier `id` makes of its name and `key`
/// makes of its document.
pub fn contribution<T: Definition>(
    loaded: FileLoad<T>,
    id: fn(&str) -> bedrock_addon_core::id::Id<T::Kind>,
    key: fn(&Document<T>) -> bedrock_addon_core::id::Id<T::Kind>,
) -> Parsed<T> {
    match loaded.result {
        Ok(document) if loaded.registered => {
            Parsed::Contribution(Contribution::Stable(key(&document), document))
        }
        Ok(document) => Parsed::Unregistered(document.diagnostics().clone()),
        Err(rejected) => Parsed::Contribution(Contribution::Rejected(
            loaded.identifier.as_deref().map(id),
            rejected,
        )),
    }
}

/// A `.json` file, the extension compared exactly (`a.JSON` is not one).
pub fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// The registry of the contributions in walk order, the top pack winning; the findings of a
/// document vanilla does not register are added to it as they are.
pub fn combine<T: Definition>(parsed: Vec<Parsed<T>>, stack: &StackCx<'_>) -> Registry<T> {
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    for file in parsed {
        match file {
            Parsed::Contribution(contribution) => builder.push(contribution),
            Parsed::Unregistered(diagnostics) => {
                for finding in diagnostics {
                    builder.push_diagnostic(finding);
                }
            }
        }
    }
    builder.finish()
}
