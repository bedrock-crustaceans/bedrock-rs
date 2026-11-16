//! What the manifest readers share: the sink of the findings the routing tells before a
//! reader is chosen, the findings the documents' hooks tell, the hooks both documents read with
//! (capabilities, `product_type`, the engine bounds), and the version texts.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, Span,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::pack_identity::{PackPath, PackType};
use bedrock_addon_core::schema::{ElementFailure, FromJson, read_list};
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use uuid::Uuid;

use super::capability::{CapabilityCx, CapabilityRegistry, CapabilityValidation};
use super::{Capability, Dependency, DependencyPlace, Module, comprehensive, legacy};

/// The lowest `min_engine_version` (and world-template `base_game_version`) a format-2 or
/// format-3 pack may declare.
pub(super) const LOWEST_ENGINE: SemVersion = SemVersion::new(1, 13, 0);

/// The name the capability registry the manifest is checked against is in scope under.
pub(super) const REGISTRY: &str = "capability registry";

/// Collects the findings of one manifest read of the file at `path` that no document's read
/// tells: the text's, the routing's.
pub(super) struct Sink<'p> {
    path: &'p PackPath,
    pub(super) diagnostics: Diagnostics,
}

impl<'p> Sink<'p> {
    /// A sink for the file at `path`.
    pub(super) fn new(path: &'p PackPath) -> Self {
        Self {
            path,
            diagnostics: Diagnostics::default(),
        }
    }

    /// The file the findings are of.
    pub(super) fn path(&self) -> &PackPath {
        self.path
    }

    /// Records `code` at `at`.
    pub(super) fn push(&mut self, code: Code, severity: Severity, phase: Phase, at: &JsonPath) {
        self.diagnostics.push(Diagnostic::new(
            code,
            severity,
            phase,
            Location::new(None, Some(self.path.clone()), at.clone(), None),
        ));
    }

    /// Records an error at the byte range `span` of the file (the root's JSON path): a finding
    /// on the text itself, before there is a value to walk.
    pub(super) fn error_in_text(&mut self, code: Code, span: Span) {
        self.diagnostics.push(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, Some(self.path.clone()), JsonPath::new(), Some(span)),
        ));
    }

    pub(super) fn error(&mut self, code: Code, at: &JsonPath) {
        self.push(code, Severity::Error, Phase::Parse, at);
    }

    /// An information from the crate: the game reads on in silence.
    pub(super) fn note(&mut self, code: Code, at: &JsonPath) {
        self.push(code, Severity::Info, Phase::Parse, at);
    }

    pub(super) fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

/// The errors the read has told so far, to tell whether a step added one.
pub(super) fn errors(read: &mut Read<'_>) -> usize {
    read.findings_mut()
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count()
}

/// `code` at `node`, spanning it.
pub(super) fn at(node: &Node<'_>, code: impl Into<Code>, severity: Severity) -> Diagnostic {
    let mut finding = Diagnostic::of(code)
        .severity(severity)
        .at(node.path().clone());
    finding.location.span = node.span();
    finding
}

/// The error `code` for the member `key` the object `object` lacks, spanning the object.
pub(super) fn lacking(object: &Node<'_>, key: &str, code: impl Into<Code>) -> Diagnostic {
    let mut finding = Diagnostic::error(code).at(object.path().child(key));
    finding.location.span = object.span();
    finding
}

/// The dependencies kept and where each is in the file.
#[derive(Default)]
pub(super) struct Dependencies {
    pub(super) list: Vec<Dependency>,
    pub(super) places: Vec<DependencyPlace>,
}

/// The pack type: the last module's of those the document read before (in scope), invalid
/// without one.
pub(super) fn pack_type(read: &Read<'_>) -> PackType {
    read.scope::<Vec<Module>>("modules")
        .map_or(PackType::Invalid, |modules| legacy::pack_type(modules))
}

/// `capabilities`: each element a capability name, checked against the registry in scope for
/// the pack of the header read before (the nil UUID without one) of the modules' type; a name
/// the registry does not grant stays in the manifest.
pub(super) fn capabilities(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Capability>> {
    read_list(node, read, ElementFailure::Drop, |element, read| {
        let capability = Capability::from_name(&String::from_json(element, read)?);
        check_capability(&capability, element, read);
        Some(capability)
    })
}

/// Checks one capability against the registry in scope: one it does not grant is the crate's
/// information (the capability registry is the crate's own API and the game has no pack-report
/// warning for it).
fn check_capability(capability: &Capability, node: &Node<'_>, read: &mut Read<'_>) {
    let pack_uuid = read
        .scope::<comprehensive::Header3>("header")
        .map(comprehensive::Header3::uuid)
        .or_else(|| {
            read.scope::<legacy::Header12>("header")
                .and_then(legacy::Header12::uuid)
        })
        .unwrap_or_else(Uuid::nil);
    let cx = CapabilityCx {
        capability,
        pack_uuid,
        pack_type: pack_type(read),
    };
    let granted = read
        .scope::<CapabilityRegistry>(REGISTRY)
        .is_some_and(|registry| {
            matches!(
                registry.validate(&cx),
                CapabilityValidation::Succeeded { .. }
            )
        });
    if !granted {
        read.push(
            at(
                node,
                core_codes::INVALID_CAPABILITY.with([capability.name().to_owned()]),
                Severity::Info,
            )
            .phase(Phase::Validate),
        );
    }
}

/// `metadata/product_type`: `""` or `addon`.
pub(super) fn product_type(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let product = String::from_json(node, read)?;
    if !product.is_empty() && product != "addon" {
        read.error(node, Code::from(&core_codes::INVALID_PROPERTY));
    }
    Some(product)
}

/// `false`, with an error at `node`, when `version` is above the engine's or below
/// [`LOWEST_ENGINE`]: the bounds of a format-2 or format-3 `min_engine_version` and of a
/// world template's `base_game_version`.
pub(super) fn engine_bounds(version: &SemVersion, node: &Node<'_>, read: &mut Read<'_>) -> bool {
    let engine = SemVersion::from(ENGINE_VERSION);
    let code = if *version > engine {
        core_codes::VERSION_TOO_HIGH.with([version.to_string(), engine.to_string()])
    } else if *version < LOWEST_ENGINE {
        core_codes::VERSION_TOO_LOW.with([version.to_string(), LOWEST_ENGINE.to_string()])
    } else {
        return true;
    };
    read.refuse(node, code);
    false
}

/// The element at `at` as the legacy reader's messages name it: a JSON pointer
/// (`/dependencies/0/version`).
pub(super) fn pointer(at: &JsonPath) -> String {
    format!("/{at}")
}

/// The place of the kept `dependencies[]` element `element` at `at`, with what its text says
/// that the value does not hold: whether it has an `optional` member (whatever its value), and,
/// for a `native` module, whether its `version` is a string of fewer than three numbers that
/// is no selector.
pub(super) fn dependency_place(element: &Value, at: JsonPath, native: bool) -> DependencyPlace {
    let relaxed = native
        && member(element, "version")
            .and_then(text)
            .is_some_and(|version| {
                !matches!(version, "alpha" | "beta" | "internal") && number_count(version) != 3
            });
    DependencyPlace {
        at,
        has_optional: member(element, "optional").is_some(),
        relaxed,
    }
}

/// How many numbers a version string writes before its pre-release tag or build metadata.
fn number_count(version: &str) -> usize {
    version
        .split(['-', '+'])
        .next()
        .unwrap_or_default()
        .split('.')
        .count()
}

/// The path of the root members and their members `keys`, from the outermost down.
pub(super) fn path(keys: &[&str]) -> JsonPath {
    JsonPath::of_keys(keys)
}

/// The member `key` of `value` when it is an object.
pub(super) fn member<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value.as_object()?.get(key)
}

/// The text of a string value.
pub(super) fn text(value: &Value) -> Option<&str> {
    value.as_string().map(facet_value::VString::as_str)
}

/// Vanilla's array form of a version, as jsoncpp reads it: exactly three elements, each
/// convertible to an unsigned integer (`null` is 0, `false` / `true` are 0 / 1, a non-negative
/// real is truncated), none above 65 535. `None` otherwise.
pub(super) fn version_array(value: &Value) -> Option<SemVersion> {
    // The array form `[a,b,c]` truncates reals and rejects any element above 0xFFFF; jsoncpp's
    // unsigned conversion admits null and booleans. The shared reader lives in the version
    // module.
    SemVersion::from_json_array(value.as_array()?.as_slice())
}

/// A version string of the strict form `major.minor.patch[-pre][+build]` (the any-version `*`,
/// the `beta` sentinel and partial versions are refused).
pub(super) fn strict_version_text(text: &str) -> Option<SemVersion> {
    let core = text.split(['-', '+']).next().unwrap_or_default();
    if core.split('.').count() != 3 {
        return None;
    }
    SemVersion::parse(text).filter(|version| !version.is_any())
}

/// A relaxed version string (`"2.0"`, `"1"`): partial versions are filled with zeros; the
/// any-version `*` and the `beta` sentinel are refused.
pub(super) fn relaxed_version_text(text: &str) -> Option<SemVersion> {
    if text == "beta" {
        return None;
    }
    SemVersion::parse(text).filter(|version| !version.is_any())
}
