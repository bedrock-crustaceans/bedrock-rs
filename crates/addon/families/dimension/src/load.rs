//! Loading dimensions: the family's chain parameters, the newest struct's parse and write,
//! `Content` / `Definition`, the walk of `dimensions/` and the combine rule (the first file
//! walked wins, packs bottom-up).

use super::parse::{RootLatest, clouds_constraint, height_constraints, parse_root};
use super::{Dimension, FLOOR, V1_26_50, validate};
use crate::history::{Any, DimensionFamily};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, default_path_flat,
};
use bedrock_addon_core::id::{DimensionId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom, write_json,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    EnvelopeCheck, ErrorMask, Family, Identity, InputKind, Loaded, MissingVersion, NoComponents,
    ParseCx, Parsed, StructIndex, ValidateCx, VersionStruct, Versioned, finish, parse_as_written,
    write_payload,
};
use bedrock_addon_support::puv::{puv_below_floor, shadowed};
use indexmap::IndexMap;

impl Family for DimensionFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "dimension";
    const PAYLOAD_KEY: &'static str = "minecraft:dimension";
    const DIRECTORIES: &'static [&'static str] = &["dimensions"];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = Dimension;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // The game reads the document with no minimum version, so the raw text is read at every
    // version.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    // The game reads the file as a whole document: the version is the one its JSON reader picks
    // from the text, and a `null` where a value is read counts as the wrong JSON type in the
    // cereal slices (the legacy parser below 1.21.60 keeps its own reading:
    // `parse::slice_null_rule`).
    const INPUT: InputKind = InputKind::Document;
    // The game passes no version of its own, so a file without `format_version` is rejected
    // with "no valid version passed by the caller or found in the input".
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

    /// The game reads every slice leniently: an unknown member is a warning.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Unstrict
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(dimension) = v {
            validate::validate(dimension, cx);
        }
    }
}

impl Versioned for Dimension {
    const VERSION: FormatVersion = V1_26_50;
}

impl VersionStruct for Dimension {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let rule = cx.load.null_rule;
        let root: RootLatest =
            parse_root(text, cx, rule, |root: &mut RootLatest| &mut root.extra, d)?;
        let inner = root.dimension;
        let components = cx.at.with(&["minecraft:dimension", "components"]);
        let height = inner
            .components
            .dimension_height
            .as_ref()
            .is_none_or(|height| {
                height_constraints(height, &components.child("minecraft:dimension_height"), d)
            });
        let clouds = inner.components.clouds.as_ref().is_none_or(|clouds| {
            clouds_constraint(clouds, &components.child("minecraft:clouds"), d)
        });
        if !(height && clouds) {
            return None;
        }
        Some(Dimension {
            description: inner.description,
            components: inner.components,
            source_format_version: cx.original_version,
            extra: inner.extra,
            root_extra: root.extra,
        })
    }

    /// The whole document: `format_version` (the document's own when it is 1.26.50 or newer,
    /// else 1.26.50), the payload and the other root members.
    fn write(&self) -> Result<Value, EncodeError> {
        let mut payload = facet_value::VObject::new();
        payload.insert("description", write_payload(&self.description)?);
        let mut components = write_payload(&self.components)?;
        if let Some(object) = components.as_object_mut()
            && self
                .components
                .dimension_height
                .as_ref()
                .is_some_and(|height| !height.is_writable())
        {
            object.remove("minecraft:dimension_height");
        }
        payload.insert("components", components);
        for (key, value) in self.extra.iter() {
            payload.insert(key, value.clone());
        }
        let mut root = facet_value::VObject::new();
        root.insert(
            "format_version",
            self.source_format_version.max(V1_26_50).to_string(),
        );
        root.insert("minecraft:dimension", payload);
        for (key, value) in self.root_extra.iter() {
            root.insert(key, value.clone());
        }
        Ok(root.into())
    }
}

impl bedrock_addon_core::__private::Sealed for Dimension {}

impl Content for Dimension {
    const NAME: &'static str = "dimension";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.write()?, out, style)
    }
}

impl JsonContent for Dimension {}

impl Definition for Dimension {
    type Kind = kind::Dimension;
    type File = Dimension;
    type Beta = NoBeta<kind::Dimension>;
}

impl Placed for Dimension {
    /// `dimensions/<name>.json` for a `minecraft:` identifier, `dimensions/<namespace>_<name>.json`
    /// for any other, the `/` of the name as `_`: the walk reads the files directly in the
    /// folder only.
    fn default_path(&self) -> PackPath {
        default_path_flat("dimensions", &self.description.identifier)
    }
}

impl Dimension {
    /// Parses one dimension file without upgrading it: the struct its `format_version` selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        let Some(text) = text(bytes) else {
            return Err(reject(origin));
        };
        let parsed = parse_as_written::<DimensionFamily>(JsonSlice::new(text), cx);
        let mut diagnostics = parsed.diagnostics;
        puv_below_floor(&mut diagnostics, parsed.version.written);
        diagnostics.locate(&origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }

    /// The warning the game gives when it cannot create the custom dimension `document`
    /// defines at world start, when the runtime fails to register it: `InvalidValue` (warning)
    /// "Dimension '{}' from pack id '{}' was invalid and could not be created!" with the
    /// identifier and the pack's UUID, at the document's identifier. The game writes the line
    /// at the content log's warning level. The failure is the runtime's, which no document
    /// rule predicts; a host that creates dimensions reports it with this.
    pub fn creation_failed(document: &Document<Dimension>) -> Diagnostic {
        let origin = document.origin();
        let at = JsonPath::new().with(&["minecraft:dimension", "description", "identifier"]);
        // The game logs this at the content log's warning level.
        Diagnostic::warn(DiagnosticCode::InvalidValue {
            text: Some(bedrock_addon_core::diagnostic::VanillaText::Formatted {
                template: "Dimension '{}' from pack id '{}' was invalid and could not be created!",
                args: vec![
                    document.description.identifier.as_str().to_owned(),
                    origin.pack.id().uuid.to_string(),
                ],
            }),
        })
        .at(at)
        .phase(Phase::Validate)
        .in_file(origin)
    }
}

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<Dimension>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let identifier = match &value {
        Any::V1_18_0(d) => d.description.identifier.as_str().to_owned(),
        Any::V1_26_50(d) => d.description.identifier.as_str().to_owned(),
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<DimensionFamily>::new(
        Some(value),
        version,
        written,
        Identity {
            identifier: Some(identifier),
            runtime_identifier: None,
        },
        bedrock_addon_core::vanilla::ActorType::Mob,
        false,
        JsonPath::new(),
        diagnostics,
    );
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(dimension)) => Ok(Document::new(
            dimension,
            origin,
            outcome.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The text of a file, after the BOM and the depth check.
fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// The rejection of an unreadable file.
fn reject(origin: Origin) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    }));
    Rejected::located(origin, diagnostics)
}

/// Loads one dimension file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<Dimension>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::file(pack, side, path.clone());
    let Some(text) = text(bytes) else {
        return Err(reject(origin));
    };
    let parsed = parse_as_written::<DimensionFamily>(JsonSlice::new(text), cx);
    let written = parsed.version.written;
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    puv_below_floor(&mut diagnostics, written);
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(dimension)) => Ok(Document::new(
            dimension,
            origin,
            outcome.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// A dimension file: an extension exactly `json` (vanilla's byte compare).
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `dimensions/*.json` (not recursive), from the last stack entry to the first: packs
/// bottom-up.
const WALK: Walk = Walk {
    directories: &["dimensions"],
    depth: ListDepth::Children,
    accept: Accept::Fn(is_json),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<Dimension> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(written_identifier(&file.bytes), rejected),
    }
}

/// `minecraft:dimension/description/identifier` of a file, when it is a string: what a file
/// that failed still names (for the script-registration check of the combine step).
fn written_identifier(bytes: &[u8]) -> Option<DimensionId> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    // Of a member written twice, the last copy is the one that stands.
    let value = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text(bytes)?),
    )
    .ok()?;
    let identifier = value
        .as_object()?
        .get("minecraft:dimension")?
        .as_object()?
        .get("description")?
        .as_object()?
        .get("identifier")?
        .as_string()?;
    Some(DimensionId::parse(identifier.as_str().to_owned()))
}

/// Whether `id` is a custom dimension registered from script (`DimensionRegistry.
/// registerCustomDimension`): a dimension `LoadOptions::built_ins` knows outside the
/// `minecraft` namespace, which the script path reserves.
fn registered_from_script(id: &DimensionId, stack: &StackCx<'_>) -> bool {
    id.namespace() != Some("minecraft")
        && stack.options.built_ins.as_ref().is_some_and(|built_ins| {
            built_ins.contains(
                <kind::Dimension as bedrock_addon_core::id::Kind>::NAME,
                id.as_str(),
            )
        })
}

/// The warning of a pack definition that replaces a script registration: `ShadowedDefinition`
/// (the registration is the definition shadowed) with vanilla's text, the identifier and the
/// pack id as arguments, at the document's identifier. The game logs it at the content log's
/// warning level.
fn replaced_registration(id: &DimensionId, origin: &Origin) -> Diagnostic {
    let at = JsonPath::new().with(&["minecraft:dimension", "description", "identifier"]);
    // The game logs this at the content log's warning level.
    Diagnostic::warn(DiagnosticCode::ShadowedDefinition {
        text: Some(bedrock_addon_core::diagnostic::VanillaText::Formatted {
            template: "Dimension definition identifier '{}' registered via scripting was replaced by the definition from pack id '{}'",
            args: vec![id.as_str().to_owned(), origin.pack.id().uuid.to_string()],
        }),
    })
    .at(at)
    .phase(Phase::Validate)
    .in_file(origin)
}

/// The registry: the first file walked wins (an insert only when the identifier is absent), so
/// the lowest-priority pack's; a later one is `ShadowedDefinition` (info; vanilla turns it into
/// a load error without text). Across packs `BottomWins` decides; within one pack the later file
/// is dropped here. The definition that wins replaces a script registration of its
/// identifier, with a warning; a file that fails to load replaces nothing.
fn combine(parsed: Vec<Contribution<Dimension>>, stack: &StackCx<'_>) -> Registry<Dimension> {
    let mut builder = RegistryBuilder::new(
        CombineRule::BottomWins {
            shadowed: Some(Notice::shadowed()),
        },
        CombineRule::bottom_wins(),
        stack.options,
    );
    let mut first: IndexMap<DimensionId, Option<usize>> = IndexMap::new();
    for contribution in parsed {
        if let Contribution::Stable(id, document) = &contribution {
            let pack = document.origin().pack.index();
            match first.get(id) {
                Some(&claimed) if claimed == pack => {
                    for finding in document.diagnostics().iter().cloned() {
                        builder.push_diagnostic(finding);
                    }
                    builder.push_diagnostic(shadowed(document.origin()));
                    continue;
                }
                Some(_) => {}
                None => {
                    first.insert(id.clone(), pack);
                    if registered_from_script(id, stack) {
                        builder.push_diagnostic(replaced_registration(id, document.origin()));
                    }
                }
            }
        }
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.dimensions()`.
pub const fn load_dimensions() -> CollectionLoader<Registry<Dimension>, Contribution<Dimension>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
